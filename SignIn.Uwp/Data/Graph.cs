using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using CommunityToolkit.Authentication;
using CommunityToolkit.Graph.Extensions;
using Microsoft.Graph;
using SignIn.Logic.Data;

namespace SignIn.Uwp.Data
{
    public class Graph
    {
        readonly string[] Scopes = { "User.Read", "Contacts.ReadWrite" };

        public Graph()
        {
            ProviderManager.Instance.GlobalProvider = new WindowsProvider(Scopes);
        }

        public bool IsAuthenticated => ProviderManager.Instance.GlobalProvider.State == ProviderState.SignedIn;

        public GraphServiceClient Client { get; private set; }

        public async Task<bool> Login()
        {
            try
            {
                if (string.IsNullOrEmpty(await ProviderManager.Instance.GlobalProvider.GetTokenAsync(silentOnly: false)))
                    return false;

                Client = ProviderManager.Instance.GlobalProvider.GetClient();
                return true;
            }
            catch (Exception ex)
            {
                await DataProvider.Current.ShowMessage("Could not log in. Check the internet connection.", ex);
                return false;
            }
        }

        #region Contacts

        public async Task<IEnumerable<Contact>> GetContacts()
        {
            return await Task.Run(async () =>
            {
                if (!IsAuthenticated)
                {
                    await ((UwpDataProvider)DataProvider.Current).ShowAuthenticationMessage();
                    return new List<Contact>();
                }

                var contacts = new List<Contact>();
                var currentRequest = Client.Me.Contacts
                    .Request()
                    .Select("id,givenName,surname,birthday,homeAddress,personalNotes");

                while (currentRequest != null)
                {
                    var nextContacts = await currentRequest.GetAsync();
                    contacts.AddRange(nextContacts);
                    currentRequest = nextContacts.NextPageRequest;
                }

                return contacts;
            });
        }

        public async Task<Contact> SaveContact(Contact contact)
        {
            try
            {
                if (!IsAuthenticated)
                    throw new InvalidOperationException("You must be logged in to save a contact!");

                return await Client.Me.Contacts.Request().AddAsync(contact);
            }
            catch (Exception ex)
            {
                throw new Exception($"Contact save failed!", ex);
            }
        }

        public async Task<Contact> UpdateContact(Contact contact)
        {
            try
            {
                if (!IsAuthenticated)
                    throw new InvalidOperationException("You must be logged in to save a contact!");

                return await Client.Me.Contacts[contact.Id].Request().UpdateAsync(contact);
            }
            catch (Exception ex)
            {
                throw new Exception($"Contact update failed!", ex);
            }
        }

        #endregion
    }
}
